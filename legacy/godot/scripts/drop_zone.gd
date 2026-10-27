extends PanelContainer
var receive_card: Callable
func _can_drop_data(_at_position: Vector2, data: Variant) -> bool:
	return data is Dictionary and data.has("card_id") and receive_card.is_valid()
func _drop_data(_at_position: Vector2, data: Variant) -> void:
	receive_card.call(str(data.card_id))
